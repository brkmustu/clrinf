using System;
using System.Collections.Generic;
using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Security.Cryptography;
using System.Text;
using Microsoft.Extensions.Options;
using Microsoft.IdentityModel.Tokens;
using EticaretApp.Application.Services.Security.JWT;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Infrastructure.Security.JWT;

public class JwtHelper : ITokenHelper
{
    private readonly TokenOptions _tokenOptions;

    public JwtHelper(IOptions<TokenOptions> tokenOptions)
    {
        _tokenOptions = tokenOptions.Value;
    }

    public AccessToken CreateToken(User user, IList<OperationClaim> operationClaims)
    {
        var securityKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(_tokenOptions.SecurityKey));
        var signingCredentials = new SigningCredentials(securityKey, SecurityAlgorithms.HmacSha512Signature);

        var claims = setClaims(user, operationClaims);
        var expiration = DateTime.UtcNow.AddMinutes(_tokenOptions.AccessTokenExpiration);

        var jwtSecurityToken = new JwtSecurityToken(
            issuer: _tokenOptions.Issuer,
            audience: _tokenOptions.Audience,
            claims: claims,
            notBefore: DateTime.UtcNow,
            expires: expiration,
            signingCredentials: signingCredentials
        );

        var handler = new JwtSecurityTokenHandler();
        var token = handler.WriteToken(jwtSecurityToken);

        return new AccessToken
        {
            Token = token,
            Expiration = expiration
        };
    }

    public RefreshToken CreateRefreshToken(User user, string ipAddress)
    {
        var randomNumber = new byte[32];
        using var rng = RandomNumberGenerator.Create();
        rng.GetBytes(randomNumber);

        return new RefreshToken
        {
            UserId = user.Id,
            Token = Convert.ToBase64String(randomNumber),
            Expires = DateTime.UtcNow.AddDays(_tokenOptions.RefreshTokenTTL),
            Created = DateTime.UtcNow,
            CreatedByIp = ipAddress
        };
    }

    private IEnumerable<Claim> setClaims(User user, IList<OperationClaim> operationClaims)
    {
        var claims = new List<Claim>
        {
            new Claim(ClaimTypes.NameIdentifier, user.Id.ToString()),
            new Claim(ClaimTypes.Email, user.Email),
            new Claim(ClaimTypes.Name, $"{user.FirstName} {user.LastName}")
        };

        foreach (var claim in operationClaims)
        {
            claims.Add(new Claim(ClaimTypes.Role, claim.Name));
        }

        return claims;
    }
}
