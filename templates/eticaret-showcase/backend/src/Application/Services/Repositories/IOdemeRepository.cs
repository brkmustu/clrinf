using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface IOdemeRepository : IAsyncRepository<Odeme, int>, IRepository<Odeme, int>
{
}
