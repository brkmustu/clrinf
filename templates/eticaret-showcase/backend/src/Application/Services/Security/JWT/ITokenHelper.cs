using System.Collections.Generic;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Services.Security.JWT;

public interface ITokenHelper
{
    AccessToken CreateToken(User user, IList<OperationClaim> operationClaims);
    RefreshToken CreateRefreshToken(User user, string ipAddress);
}
