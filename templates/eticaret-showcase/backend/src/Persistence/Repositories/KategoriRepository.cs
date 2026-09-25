using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class KategoriRepository : EfRepositoryBase<Kategori, int, BaseDbContext>, IKategoriRepository
{
    public KategoriRepository(BaseDbContext context) : base(context) { }
}
