using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface IKategoriRepository : IAsyncRepository<Kategori, int>, IRepository<Kategori, int>
{
}
