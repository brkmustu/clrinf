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
using static EticaretApp.Application.Moduller.Siparisler.SiparislerOperationClaims;

namespace EticaretApp.Application.Moduller.Siparisler;

#region Query Responses & DTOs

public sealed record GetByIdSiparisResponse
{
    public int Id { get; init; }
    public string SiparisNo { get; init; }
    public string KullaniciId { get; init; }
    public decimal ToplamTutar { get; init; }
    public string Durum { get; init; }
    public string? KargoTakipNo { get; init; }
}

public sealed record GetListSiparisListItemDto
{
    public int Id { get; init; }
    public string SiparisNo { get; init; }
    public string KullaniciId { get; init; }
    public decimal ToplamTutar { get; init; }
    public string Durum { get; init; }
    public string? KargoTakipNo { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, SiparislerOperationClaims.Read)]
[Logged]
public sealed record GetByIdSiparisQuery(int Id) : IQuery<Result<GetByIdSiparisResponse, DomainError>>;

internal sealed class GetByIdSiparisQueryHandler : IQueryHandler<GetByIdSiparisQuery, Result<GetByIdSiparisResponse, DomainError>>
{
    private readonly ISiparisRepository _siparisRepository;
    private readonly SiparisBusinessRules _siparisBusinessRules;

    public GetByIdSiparisQueryHandler(
        ISiparisRepository siparisRepository,
        SiparisBusinessRules siparisBusinessRules)
    {
        _siparisRepository = siparisRepository;
        _siparisBusinessRules = siparisBusinessRules;
    }

    public async ValueTask<Result<GetByIdSiparisResponse, DomainError>> HandleAsync(
        GetByIdSiparisQuery request,
        CancellationToken cancellationToken)
    {
        Siparis? entity = await _siparisRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _siparisBusinessRules.SiparisShouldExistWhenSelected(entity);

        return Result<GetByIdSiparisResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, SiparislerOperationClaims.Read)]
[Logged]
public sealed record GetListSiparisQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListSiparisListItemDto>, DomainError>>;

internal sealed class GetListSiparisQueryHandler : IQueryHandler<GetListSiparisQuery, Result<GetListResponse<GetListSiparisListItemDto>, DomainError>>
{
    private readonly ISiparisRepository _siparisRepository;

    public GetListSiparisQueryHandler(ISiparisRepository siparisRepository)
    {
        _siparisRepository = siparisRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListSiparisListItemDto>, DomainError>> HandleAsync(
        GetListSiparisQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _siparisRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListSiparisListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
