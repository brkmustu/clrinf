using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;
using EticaretApp.Persistence.Common;
using EticaretApp.Persistence.Contexts;

namespace EticaretApp.Persistence.Repositories;

public class DegerlendirmeRepository : EfRepositoryBase<Degerlendirme, int, BaseDbContext>, IDegerlendirmeRepository
{
    public DegerlendirmeRepository(BaseDbContext context) : base(context) { }
}
