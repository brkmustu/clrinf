using EticaretApp.Application.Common.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Services.Repositories;

public interface IUserRepository : IAsyncRepository<User, int>, IRepository<User, int>
{
}
