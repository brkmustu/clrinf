using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface IStokRepository : IAsyncRepository<Stok, int>, IRepository<Stok, int>
{
}