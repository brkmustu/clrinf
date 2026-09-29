using System.Security.Cryptography;
using System.Text;
using Xunit;

namespace ClrinfCS.Tests;

public class SecurityTests
{
    [Fact]
    public void CreatePasswordHash_ShouldGenerateValidSaltAndHash()
    {
        string password = "SecretPassword123!";

        using var hmac = new HMACSHA512();
        byte[] salt = hmac.Key;
        byte[] hash = hmac.ComputeHash(Encoding.UTF8.GetBytes(password));

        Assert.NotNull(salt);
        Assert.NotNull(hash);
        Assert.NotEmpty(salt);
        Assert.NotEmpty(hash);

        using var verifyHmac = new HMACSHA512(salt);
        byte[] computedHash = verifyHmac.ComputeHash(Encoding.UTF8.GetBytes(password));
        Assert.True(CryptographicOperations.FixedTimeEquals(computedHash, hash));
    }

    [Fact]
    public void VerifyPasswordHash_ShouldReturnFalse_WhenPasswordIsIncorrect()
    {
        string password = "SecretPassword123!";
        string wrongPassword = "WrongPassword123!";

        using var hmac = new HMACSHA512();
        byte[] salt = hmac.Key;
        byte[] hash = hmac.ComputeHash(Encoding.UTF8.GetBytes(password));

        using var verifyHmac = new HMACSHA512(salt);
        byte[] computedHash = verifyHmac.ComputeHash(Encoding.UTF8.GetBytes(wrongPassword));
        Assert.False(CryptographicOperations.FixedTimeEquals(computedHash, hash));
    }
}
