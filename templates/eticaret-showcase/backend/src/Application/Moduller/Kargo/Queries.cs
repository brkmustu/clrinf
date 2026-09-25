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
using static EticaretApp.Application.Features.Kargolar.KargolarOperationClaims;

namespace EticaretApp.Application.Features.Kargolar;

#region Query Responses & DTOs

public sealed record GetByIdKargoResponse
{
    public int Id { get; init; }
}

public sealed record GetListKargoListItemDto
{
    public int Id { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, KargolarOperationClaims.Read)]
[Logged]
public sealed record GetByIdKargoQuery(int Id) : IQuery<Result<GetByIdKargoResponse, DomainError>>;

internal sealed class GetByIdKargoQueryHandler : IQueryHandler<GetByIdKargoQuery, Result<GetByIdKargoResponse, DomainError>>
{
    private readonly IKargoRepository _kargoRepository;
    private readonly KargoBusinessRules _kargoBusinessRules;

    public GetByIdKargoQueryHandler(
        IKargoRepository kargoRepository,
        KargoBusinessRules kargoBusinessRules)
    {
        _kargoRepository = kargoRepository;
        _kargoBusinessRules = kargoBusinessRules;
    }

    public async ValueTask<Result<GetByIdKargoResponse, DomainError>> HandleAsync(
        GetByIdKargoQuery request,
        CancellationToken cancellationToken)
    {
        Kargo? entity = await _kargoRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _kargoBusinessRules.KargoShouldExistWhenSelected(entity);

        return Result<GetByIdKargoResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, KargolarOperationClaims.Read)]
[Logged]
public sealed record GetListKargoQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListKargoListItemDto>, DomainError>>;

internal sealed class GetListKargoQueryHandler : IQueryHandler<GetListKargoQuery, Result<GetListResponse<GetListKargoListItemDto>, DomainError>>
{
    private readonly IKargoRepository _kargoRepository;

    public GetListKargoQueryHandler(IKargoRepository kargoRepository)
    {
        _kargoRepository = kargoRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListKargoListItemDto>, DomainError>> HandleAsync(
        GetListKargoQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _kargoRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListKargoListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
