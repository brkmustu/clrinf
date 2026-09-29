use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use clrinf_adapters::AppError;
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use rsa::{
    pkcs1::DecodeRsaPublicKey, pkcs8::DecodePublicKey, traits::PublicKeyParts, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::credentials::Principal;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub is_service_account: bool,
    pub iss: String,
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub issuer: String,
    pub audience: String,
    pub kid: String,
    pub user_ttl_seconds: u32,
    pub service_ttl_seconds: u32,
}

pub struct JwtService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    config: JwtConfig,
    jwks: Value,
}

impl JwtService {
    pub fn new(config: JwtConfig, private_pem: &[u8], public_pem: &[u8]) -> Result<Self> {
        if [&config.issuer, &config.audience, &config.kid]
            .iter()
            .any(|v| v.trim().is_empty())
            || config.user_ttl_seconds == 0
            || config.service_ttl_seconds == 0
            || config.user_ttl_seconds > 86400
            || config.service_ttl_seconds > 86400
        {
            bail!("JWT issuer, audience, kid and TTLs (1..86400 seconds) are required");
        }
        let pem = std::str::from_utf8(public_pem).context("invalid public key PEM")?;
        let public = RsaPublicKey::from_public_key_pem(pem)
            .or_else(|_| RsaPublicKey::from_pkcs1_pem(pem))
            .context("public key must be an RSA public PEM key")?;
        if public.n().bits() < 2048 {
            bail!("RSA keys must be at least 2048 bits");
        }
        let service = Self {
            encoding: EncodingKey::from_rsa_pem(private_pem)
                .context("invalid RSA private PEM key")?,
            decoding: DecodingKey::from_rsa_pem(public_pem)
                .context("invalid RSA public PEM key")?,
            jwks: json!({"keys": [{
                "kty": "RSA", "use": "sig", "alg": "RS256", "kid": config.kid,
                "n": URL_SAFE_NO_PAD.encode(public.n().to_bytes_be()),
                "e": URL_SAFE_NO_PAD.encode(public.e().to_bytes_be()),
            }]}),
            config,
        };
        // Fail startup if the two configured keys do not form a signing/verification pair.
        let claims = service.claims(
            &Principal {
                id: "key-validation".into(),
                roles: vec![],
                permissions: vec![],
                is_service_account: false,
                profile: None,
            },
            "key-validation",
        );
        let token = service
            .encode(&claims)
            .map_err(|_| anyhow::anyhow!("RSA key signing failed"))?;
        service
            .verify(&token, "key-validation")
            .map_err(|_| anyhow::anyhow!("RSA public and private keys do not match"))?;
        Ok(service)
    }

    pub fn ttl(&self, is_service: bool) -> u32 {
        if is_service {
            self.config.service_ttl_seconds
        } else {
            self.config.user_ttl_seconds
        }
    }

    fn claims(&self, principal: &Principal, tenant: &str) -> Claims {
        let now = Utc::now().timestamp() as usize;
        Claims {
            sub: principal.id.clone(),
            tenant_id: tenant.into(),
            roles: principal.roles.clone(),
            permissions: principal.permissions.clone(),
            is_service_account: principal.is_service_account,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            exp: now + self.ttl(principal.is_service_account) as usize,
            iat: now,
        }
    }

    fn encode(&self, claims: &Claims) -> Result<String, AppError> {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(self.config.kid.clone());
        encode(&header, claims, &self.encoding)
            .map_err(|e| AppError::Internal(format!("JWT signing failed: {e}")))
    }

    pub fn issue(&self, principal: &Principal, tenant: &str) -> Result<String, AppError> {
        self.encode(&self.claims(principal, tenant))
    }

    pub fn verify(&self, token: &str, tenant: &str) -> Result<Claims, AppError> {
        let invalid = || AppError::Unauthorized("Invalid token".into());
        let header = decode_header(token).map_err(|_| invalid())?;
        if header.alg != Algorithm::RS256 || header.kid.as_deref() != Some(&self.config.kid) {
            return Err(invalid());
        }
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        validation.set_required_spec_claims(&["exp", "iat", "iss", "aud", "sub"]);
        validation.leeway = 0;
        let claims = decode::<Claims>(token, &self.decoding, &validation)
            .map_err(|_| invalid())?
            .claims;
        let now = Utc::now().timestamp() as usize;
        if claims.tenant_id != tenant
            || claims.sub.trim().is_empty()
            || claims.iat > now
            || claims.exp <= now
            || claims.exp <= claims.iat
        {
            return Err(invalid());
        }
        Ok(claims)
    }

    pub fn jwks(&self) -> &Value {
        &self.jwks
    }
}
