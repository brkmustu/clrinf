using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class SepetRepository : EfRepositoryBase<Sepet, int, BaseDbContext>, ISepetRepository
{
    public SepetRepository(BaseDbContext context) : base(context)
    {
    }
}