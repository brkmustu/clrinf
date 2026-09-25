using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface IKuponRepository : IAsyncRepository<Kupon, int>, IRepository<Kupon, int>
{
}
