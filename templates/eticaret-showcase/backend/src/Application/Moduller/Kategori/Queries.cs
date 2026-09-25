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
using static EticaretApp.Application.Features.Kategoriler.KategorilerOperationClaims;

namespace EticaretApp.Application.Features.Kategoriler;

#region Query Responses & DTOs

public sealed record GetByIdKategoriResponse
{
    public int Id { get; init; }
    public string Ad { get; init; } = string.Empty;
    public string Slug { get; init; } = string.Empty;
    public int Sira { get; init; }
    public bool Aktif { get; init; }
}

public sealed record GetListKategoriListItemDto
{
    public int Id { get; init; }
    public string Ad { get; init; } = string.Empty;
    public string Slug { get; init; } = string.Empty;
    public int Sira { get; init; }
    public bool Aktif { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, KategorilerOperationClaims.Read)]
[Logged]
public sealed record GetByIdKategoriQuery(int Id) : IQuery<Result<GetByIdKategoriResponse, DomainError>>;

internal sealed class GetByIdKategoriQueryHandler : IQueryHandler<GetByIdKategoriQuery, Result<GetByIdKategoriResponse, DomainError>>
{
    private readonly IKategoriRepository _kategoriRepository;
    private readonly KategoriBusinessRules _kategoriBusinessRules;

    public GetByIdKategoriQueryHandler(
        IKategoriRepository kategoriRepository,
        KategoriBusinessRules kategoriBusinessRules)
    {
        _kategoriRepository = kategoriRepository;
        _kategoriBusinessRules = kategoriBusinessRules;
    }

    public async ValueTask<Result<GetByIdKategoriResponse, DomainError>> HandleAsync(
        GetByIdKategoriQuery request,
        CancellationToken cancellationToken)
    {
        Kategori? entity = await _kategoriRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _kategoriBusinessRules.KategoriShouldExistWhenSelected(entity);

        return Result<GetByIdKategoriResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, KategorilerOperationClaims.Read)]
[Logged]
public sealed record GetListKategoriQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListKategoriListItemDto>, DomainError>>;

internal sealed class GetListKategoriQueryHandler : IQueryHandler<GetListKategoriQuery, Result<GetListResponse<GetListKategoriListItemDto>, DomainError>>
{
    private readonly IKategoriRepository _kategoriRepository;

    public GetListKategoriQueryHandler(IKategoriRepository kategoriRepository)
    {
        _kategoriRepository = kategoriRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListKategoriListItemDto>, DomainError>> HandleAsync(
        GetListKategoriQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _kategoriRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListKategoriListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
