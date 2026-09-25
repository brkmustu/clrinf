using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface IUrunRepository : IAsyncRepository<Urun, int>, IRepository<Urun, int>
{
}