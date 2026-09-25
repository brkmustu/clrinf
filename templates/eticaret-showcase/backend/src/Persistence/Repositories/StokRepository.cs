using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class StokRepository : EfRepositoryBase<Stok, int, BaseDbContext>, IStokRepository
{
    public StokRepository(BaseDbContext context) : base(context)
    {
    }
}