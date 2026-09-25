using EticaretApp.Application.Common.Pipeline;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.DependencyInjection;

namespace EticaretApp.WebAPI.Controllers;

[ApiController]
public abstract class BaseController : ControllerBase
{
    private IDispatcher? _dispatcher;
    protected IDispatcher Dispatcher => _dispatcher ??= HttpContext.RequestServices.GetRequiredService<IDispatcher>();
}
