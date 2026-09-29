use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Claims {
    sub: String,
    tenant_id: String,
    roles: Vec<String>,
    permissions: Vec<String>,
    is_service_account: bool,
    iss: String,
    exp: usize,
    iat: usize,
}

#[test]
fn test_jwt_issuance_and_verification() {
    let secret = "test-secret-key-1234567890";
    let now = Utc::now();
    let exp = now + Duration::hours(1);

    let claims = Claims {
        sub: "user-acme".to_string(),
        tenant_id: "tenant-acme".to_string(),
        roles: vec!["User".to_string()],
        permissions: vec!["stok:read".to_string(), "stok:reserve".to_string()],
        is_service_account: false,
        iss: "clrinf/auth-service".to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .expect("Token encoding failed");

    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .expect("Token decoding failed");

    assert_eq!(decoded.claims.sub, "user-acme");
    assert_eq!(decoded.claims.tenant_id, "tenant-acme");
    assert!(decoded
        .claims
        .permissions
        .contains(&"stok:reserve".to_string()));
}
