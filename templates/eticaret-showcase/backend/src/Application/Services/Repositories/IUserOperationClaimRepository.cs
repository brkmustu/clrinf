using EticaretApp.Application.Common.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Services.Repositories;

public interface IUserOperationClaimRepository : IAsyncRepository<UserOperationClaim, int>, IRepository<UserOperationClaim, int>
{
}
