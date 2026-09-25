using EticaretApp.Application.Moduller.Stoklar;
using EticaretApp.Application.Common.Requests;
using Microsoft.AspNetCore.Mvc;

namespace EticaretApp.WebAPI.Controllers;

[Route("api/[controller]")]
[ApiController]
public class StokController : BaseController
{
    [HttpPost]
    public async Task<IActionResult> Add([FromBody] CreateStokCommand command)
    {
        var result = await Dispatcher.SendAsync(command);
        return result.ToActionResult();
    }

    [HttpPut]
    public async Task<IActionResult> Update([FromBody] UpdateStokCommand command)
    {
        var result = await Dispatcher.SendAsync(command);
        return result.ToActionResult();
    }

    [HttpDelete("{id}")]
    public async Task<IActionResult> Delete([FromRoute] int id)
    {
        DeleteStokCommand command = new(id);
        var result = await Dispatcher.SendAsync(command);
        return result.ToActionResult();
    }

    [HttpGet("{id}")]
    public async Task<IActionResult> GetById([FromRoute] int id)
    {
        GetByIdStokQuery query = new(id);
        var result = await Dispatcher.QueryAsync(query);
        return result.ToActionResult();
    }

    [HttpGet]
    public async Task<IActionResult> GetList([FromQuery] PageRequest pageRequest)
    {
        GetListStokQuery query = new(pageRequest);
        var result = await Dispatcher.QueryAsync(query);
        return result.ToActionResult();
    }
}