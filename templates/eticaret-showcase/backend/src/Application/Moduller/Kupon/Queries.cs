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
using static EticaretApp.Application.Features.Kuponlar.KuponlarOperationClaims;

namespace EticaretApp.Application.Features.Kuponlar;

#region Query Responses & DTOs

public sealed record GetByIdKuponResponse
{
    public int Id { get; init; }
}

public sealed record GetListKuponListItemDto
{
    public int Id { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, KuponlarOperationClaims.Read)]
[Logged]
public sealed record GetByIdKuponQuery(int Id) : IQuery<Result<GetByIdKuponResponse, DomainError>>;

internal sealed class GetByIdKuponQueryHandler : IQueryHandler<GetByIdKuponQuery, Result<GetByIdKuponResponse, DomainError>>
{
    private readonly IKuponRepository _kuponRepository;
    private readonly KuponBusinessRules _kuponBusinessRules;

    public GetByIdKuponQueryHandler(
        IKuponRepository kuponRepository,
        KuponBusinessRules kuponBusinessRules)
    {
        _kuponRepository = kuponRepository;
        _kuponBusinessRules = kuponBusinessRules;
    }

    public async ValueTask<Result<GetByIdKuponResponse, DomainError>> HandleAsync(
        GetByIdKuponQuery request,
        CancellationToken cancellationToken)
    {
        Kupon? entity = await _kuponRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _kuponBusinessRules.KuponShouldExistWhenSelected(entity);

        return Result<GetByIdKuponResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, KuponlarOperationClaims.Read)]
[Logged]
public sealed record GetListKuponQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListKuponListItemDto>, DomainError>>;

internal sealed class GetListKuponQueryHandler : IQueryHandler<GetListKuponQuery, Result<GetListResponse<GetListKuponListItemDto>, DomainError>>
{
    private readonly IKuponRepository _kuponRepository;

    public GetListKuponQueryHandler(IKuponRepository kuponRepository)
    {
        _kuponRepository = kuponRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListKuponListItemDto>, DomainError>> HandleAsync(
        GetListKuponQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _kuponRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListKuponListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
