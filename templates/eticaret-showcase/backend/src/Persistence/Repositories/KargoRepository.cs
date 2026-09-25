using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class KargoRepository : EfRepositoryBase<Kargo, int, BaseDbContext>, IKargoRepository
{
    public KargoRepository(BaseDbContext context) : base(context) { }
}
