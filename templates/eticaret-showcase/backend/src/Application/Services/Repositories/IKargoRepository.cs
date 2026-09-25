using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Repositories;

namespace EticaretApp.Application.Services.Repositories;

public interface IKargoRepository : IAsyncRepository<Kargo, int>, IRepository<Kargo, int>
{
}
