using Microsoft.AspNetCore.Mvc;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Domain.Common.Exceptions;

namespace EticaretApp.WebAPI.Controllers;

public static class ResultHttpExtensions
{
    public static IActionResult ToActionResult<T>(this Result<T, DomainError> result) =>
        result.Match<IActionResult>(
            onSuccess: value => new OkObjectResult(value),
            onFailure: error => error switch
            {
                DomainError.Validation v => new BadRequestObjectResult(new { v.Code, v.Message, v.Field, v.Reason }),
                DomainError.NotFound nf  => new NotFoundObjectResult(new { nf.Code, nf.Message, nf.Entity, nf.Id }),
                DomainError.Conflict c   => new ConflictObjectResult(new { c.Code, c.Message, c.Reason }),
                _ => new ObjectResult(new { error.Code, error.Message }) { StatusCode = 500 }
            });
}
