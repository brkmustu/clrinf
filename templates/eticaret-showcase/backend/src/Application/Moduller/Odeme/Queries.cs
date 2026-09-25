using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Common.Requests;
using EticaretApp.Application.Common.Responses;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Features.Odemeler.OdemelerOperationClaims;

namespace EticaretApp.Application.Features.Odemeler;

#region Query Responses & DTOs

public sealed record GetByIdOdemeResponse
{
    public int Id { get; init; }
}

public sealed record GetListOdemeListItemDto
{
    public int Id { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, OdemelerOperationClaims.Read)]
[Logged]
public sealed record GetByIdOdemeQuery(int Id) : IQuery<Result<GetByIdOdemeResponse, DomainError>>;

internal sealed class GetByIdOdemeQueryHandler : IQueryHandler<GetByIdOdemeQuery, Result<GetByIdOdemeResponse, DomainError>>
{
    private readonly IOdemeRepository _odemeRepository;
    private readonly OdemeBusinessRules _odemeBusinessRules;

    public GetByIdOdemeQueryHandler(
        IOdemeRepository odemeRepository,
        OdemeBusinessRules odemeBusinessRules)
    {
        _odemeRepository = odemeRepository;
        _odemeBusinessRules = odemeBusinessRules;
    }

    public async ValueTask<Result<GetByIdOdemeResponse, DomainError>> HandleAsync(
        GetByIdOdemeQuery request,
        CancellationToken cancellationToken)
    {
        Odeme? entity = await _odemeRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _odemeBusinessRules.OdemeShouldExistWhenSelected(entity);

        return Result<GetByIdOdemeResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, OdemelerOperationClaims.Read)]
[Logged]
public sealed record GetListOdemeQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListOdemeListItemDto>, DomainError>>;

internal sealed class GetListOdemeQueryHandler : IQueryHandler<GetListOdemeQuery, Result<GetListResponse<GetListOdemeListItemDto>, DomainError>>
{
    private readonly IOdemeRepository _odemeRepository;

    public GetListOdemeQueryHandler(IOdemeRepository odemeRepository)
    {
        _odemeRepository = odemeRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListOdemeListItemDto>, DomainError>> HandleAsync(
        GetListOdemeQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _odemeRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListOdemeListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
