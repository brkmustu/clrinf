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
using static EticaretApp.Application.Moduller.Urunlar.UrunlarOperationClaims;

namespace EticaretApp.Application.Moduller.Urunlar;

#region Query Responses & DTOs

public sealed record GetByIdUrunResponse
{
    public int Id { get; init; }
    public string Sku { get; init; }
    public string Baslik { get; init; }
    public string? Aciklama { get; init; }
    public decimal BirimFiyat { get; init; }
    public string ParaBirimi { get; init; }
    public string KategoriYolu { get; init; }
    public int MusaitStok { get; init; }
    public bool Yayinda { get; init; }
}

public sealed record GetListUrunListItemDto
{
    public int Id { get; init; }
    public string Sku { get; init; }
    public string Baslik { get; init; }
    public string? Aciklama { get; init; }
    public decimal BirimFiyat { get; init; }
    public string ParaBirimi { get; init; }
    public string KategoriYolu { get; init; }
    public int MusaitStok { get; init; }
    public bool Yayinda { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, UrunlarOperationClaims.Read)]
[Logged]
public sealed record GetByIdUrunQuery(int Id) : IQuery<Result<GetByIdUrunResponse, DomainError>>;

internal sealed class GetByIdUrunQueryHandler : IQueryHandler<GetByIdUrunQuery, Result<GetByIdUrunResponse, DomainError>>
{
    private readonly IUrunRepository _urunRepository;
    private readonly UrunBusinessRules _urunBusinessRules;

    public GetByIdUrunQueryHandler(
        IUrunRepository urunRepository,
        UrunBusinessRules urunBusinessRules)
    {
        _urunRepository = urunRepository;
        _urunBusinessRules = urunBusinessRules;
    }

    public async ValueTask<Result<GetByIdUrunResponse, DomainError>> HandleAsync(
        GetByIdUrunQuery request,
        CancellationToken cancellationToken)
    {
        Urun? entity = await _urunRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _urunBusinessRules.UrunShouldExistWhenSelected(entity);

        return Result<GetByIdUrunResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, UrunlarOperationClaims.Read)]
[Logged]
public sealed record GetListUrunQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListUrunListItemDto>, DomainError>>;

internal sealed class GetListUrunQueryHandler : IQueryHandler<GetListUrunQuery, Result<GetListResponse<GetListUrunListItemDto>, DomainError>>
{
    private readonly IUrunRepository _urunRepository;

    public GetListUrunQueryHandler(IUrunRepository urunRepository)
    {
        _urunRepository = urunRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListUrunListItemDto>, DomainError>> HandleAsync(
        GetListUrunQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _urunRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListUrunListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
