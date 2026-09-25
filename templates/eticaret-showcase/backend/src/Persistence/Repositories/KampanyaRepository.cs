using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class KampanyaRepository : EfRepositoryBase<Kampanya, int, BaseDbContext>, IKampanyaRepository
{
    public KampanyaRepository(BaseDbContext context) : base(context)
    {
    }
}