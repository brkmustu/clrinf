using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface ISiparisRepository : IAsyncRepository<Siparis, int>, IRepository<Siparis, int>
{
}