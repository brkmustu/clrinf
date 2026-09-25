using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class UrunRepository : EfRepositoryBase<Urun, int, BaseDbContext>, IUrunRepository
{
    public UrunRepository(BaseDbContext context) : base(context)
    {
    }
}