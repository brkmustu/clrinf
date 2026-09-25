using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface ISepetRepository : IAsyncRepository<Sepet, int>, IRepository<Sepet, int>
{
}