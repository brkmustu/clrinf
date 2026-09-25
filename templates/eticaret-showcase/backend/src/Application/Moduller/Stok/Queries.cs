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
using static EticaretApp.Application.Moduller.Stoklar.StoklarOperationClaims;

namespace EticaretApp.Application.Moduller.Stoklar;

#region Query Responses & DTOs

public sealed record GetByIdStokResponse
{
    public int Id { get; init; }
    public string Sku { get; init; }
    public int Miktar { get; init; }
    public int RezerveMiktar { get; init; }
}

public sealed record GetListStokListItemDto
{
    public int Id { get; init; }
    public string Sku { get; init; }
    public int Miktar { get; init; }
    public int RezerveMiktar { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, StoklarOperationClaims.Read)]
[Logged]
public sealed record GetByIdStokQuery(int Id) : IQuery<Result<GetByIdStokResponse, DomainError>>;

internal sealed class GetByIdStokQueryHandler : IQueryHandler<GetByIdStokQuery, Result<GetByIdStokResponse, DomainError>>
{
    private readonly IStokRepository _stokRepository;
    private readonly StokBusinessRules _stokBusinessRules;

    public GetByIdStokQueryHandler(
        IStokRepository stokRepository,
        StokBusinessRules stokBusinessRules)
    {
        _stokRepository = stokRepository;
        _stokBusinessRules = stokBusinessRules;
    }

    public async ValueTask<Result<GetByIdStokResponse, DomainError>> HandleAsync(
        GetByIdStokQuery request,
        CancellationToken cancellationToken)
    {
        Stok? entity = await _stokRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _stokBusinessRules.StokShouldExistWhenSelected(entity);

        return Result<GetByIdStokResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, StoklarOperationClaims.Read)]
[Logged]
public sealed record GetListStokQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListStokListItemDto>, DomainError>>;

internal sealed class GetListStokQueryHandler : IQueryHandler<GetListStokQuery, Result<GetListResponse<GetListStokListItemDto>, DomainError>>
{
    private readonly IStokRepository _stokRepository;

    public GetListStokQueryHandler(IStokRepository stokRepository)
    {
        _stokRepository = stokRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListStokListItemDto>, DomainError>> HandleAsync(
        GetListStokQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _stokRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListStokListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
