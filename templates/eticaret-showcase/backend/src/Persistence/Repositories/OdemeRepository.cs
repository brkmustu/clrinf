using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class OdemeRepository : EfRepositoryBase<Odeme, int, BaseDbContext>, IOdemeRepository
{
    public OdemeRepository(BaseDbContext context) : base(context) { }
}
