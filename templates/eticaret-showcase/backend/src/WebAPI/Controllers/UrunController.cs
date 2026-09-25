using EticaretApp.Application.Moduller.Urunlar;
using EticaretApp.Application.Common.Requests;
using Microsoft.AspNetCore.Mvc;

namespace EticaretApp.WebAPI.Controllers;

[Route("api/[controller]")]
[ApiController]
public class UrunController : BaseController
{
    [HttpPost]
    public async Task<IActionResult> Add([FromBody] CreateUrunCommand command)
    {
        var result = await Dispatcher.SendAsync(command);
        return result.ToActionResult();
    }

    [HttpPut]
    public async Task<IActionResult> Update([FromBody] UpdateUrunCommand command)
    {
        var result = await Dispatcher.SendAsync(command);
        return result.ToActionResult();
    }

    [HttpDelete("{id}")]
    public async Task<IActionResult> Delete([FromRoute] int id)
    {
        DeleteUrunCommand command = new(id);
        var result = await Dispatcher.SendAsync(command);
        return result.ToActionResult();
    }

    [HttpGet("{id}")]
    public async Task<IActionResult> GetById([FromRoute] int id)
    {
        GetByIdUrunQuery query = new(id);
        var result = await Dispatcher.QueryAsync(query);
        return result.ToActionResult();
    }

    [HttpGet]
    public async Task<IActionResult> GetList([FromQuery] PageRequest pageRequest)
    {
        GetListUrunQuery query = new(pageRequest);
        var result = await Dispatcher.QueryAsync(query);
        return result.ToActionResult();
    }
}