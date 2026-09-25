using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class SiparisRepository : EfRepositoryBase<Siparis, int, BaseDbContext>, ISiparisRepository
{
    public SiparisRepository(BaseDbContext context) : base(context)
    {
    }
}