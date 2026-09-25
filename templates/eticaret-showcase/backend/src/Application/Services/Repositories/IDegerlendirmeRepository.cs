using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface IDegerlendirmeRepository : IAsyncRepository<Degerlendirme, int>, IRepository<Degerlendirme, int>
{
}
