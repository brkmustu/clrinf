using EticaretApp.Application.Common.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Services.Repositories;

public interface IOperationClaimRepository : IAsyncRepository<OperationClaim, int>, IRepository<OperationClaim, int>
{
}
