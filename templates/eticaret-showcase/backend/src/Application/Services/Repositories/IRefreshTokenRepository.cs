using EticaretApp.Application.Common.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Services.Repositories;

public interface IRefreshTokenRepository : IAsyncRepository<RefreshToken, int>, IRepository<RefreshToken, int>
{
}
