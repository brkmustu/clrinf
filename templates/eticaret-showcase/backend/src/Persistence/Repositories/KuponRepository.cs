using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class KuponRepository : EfRepositoryBase<Kupon, int, BaseDbContext>, IKuponRepository
{
    public KuponRepository(BaseDbContext context) : base(context) { }
}
