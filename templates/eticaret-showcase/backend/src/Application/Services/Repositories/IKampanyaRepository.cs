using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface IKampanyaRepository : IAsyncRepository<Kampanya, int>, IRepository<Kampanya, int>
{
}