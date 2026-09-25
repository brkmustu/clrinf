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
using static EticaretApp.Application.Features.Degerlendirmeler.DegerlendirmelerOperationClaims;

namespace EticaretApp.Application.Features.Degerlendirmeler;

#region Query Responses & DTOs

public sealed record GetByIdDegerlendirmeResponse
{
    public int Id { get; init; }
    public int UrunId { get; init; }
    public string KullaniciId { get; init; } = string.Empty;
    public int Puan { get; init; }
    public string? Yorum { get; init; }
    public bool Onaylandi { get; init; }
}

public sealed record GetListDegerlendirmeListItemDto
{
    public int Id { get; init; }
    public int UrunId { get; init; }
    public string KullaniciId { get; init; } = string.Empty;
    public int Puan { get; init; }
    public string? Yorum { get; init; }
    public bool Onaylandi { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, DegerlendirmelerOperationClaims.Read)]
[Logged]
public sealed record GetByIdDegerlendirmeQuery(int Id) : IQuery<Result<GetByIdDegerlendirmeResponse, DomainError>>;

internal sealed class GetByIdDegerlendirmeQueryHandler : IQueryHandler<GetByIdDegerlendirmeQuery, Result<GetByIdDegerlendirmeResponse, DomainError>>
{
    private readonly IDegerlendirmeRepository _degerlendirmeRepository;
    private readonly DegerlendirmeBusinessRules _degerlendirmeBusinessRules;

    public GetByIdDegerlendirmeQueryHandler(
        IDegerlendirmeRepository degerlendirmeRepository,
        DegerlendirmeBusinessRules degerlendirmeBusinessRules)
    {
        _degerlendirmeRepository = degerlendirmeRepository;
        _degerlendirmeBusinessRules = degerlendirmeBusinessRules;
    }

    public async ValueTask<Result<GetByIdDegerlendirmeResponse, DomainError>> HandleAsync(
        GetByIdDegerlendirmeQuery request,
        CancellationToken cancellationToken)
    {
        Degerlendirme? entity = await _degerlendirmeRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _degerlendirmeBusinessRules.DegerlendirmeShouldExistWhenSelected(entity);

        return Result<GetByIdDegerlendirmeResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, DegerlendirmelerOperationClaims.Read)]
[Logged]
public sealed record GetListDegerlendirmeQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListDegerlendirmeListItemDto>, DomainError>>;

internal sealed class GetListDegerlendirmeQueryHandler : IQueryHandler<GetListDegerlendirmeQuery, Result<GetListResponse<GetListDegerlendirmeListItemDto>, DomainError>>
{
    private readonly IDegerlendirmeRepository _degerlendirmeRepository;

    public GetListDegerlendirmeQueryHandler(IDegerlendirmeRepository degerlendirmeRepository)
    {
        _degerlendirmeRepository = degerlendirmeRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListDegerlendirmeListItemDto>, DomainError>> HandleAsync(
        GetListDegerlendirmeQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _degerlendirmeRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListDegerlendirmeListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
