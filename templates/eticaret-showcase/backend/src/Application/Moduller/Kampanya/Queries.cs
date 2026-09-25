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
using static EticaretApp.Application.Moduller.Kampanyalar.KampanyalarOperationClaims;

namespace EticaretApp.Application.Moduller.Kampanyalar;

#region Query Responses & DTOs

public sealed record GetByIdKampanyaResponse
{
    public int Id { get; init; }
    public string Kod { get; init; }
    public string Baslik { get; init; }
    public decimal Deger { get; init; }
    public bool Aktif { get; init; }
}

public sealed record GetListKampanyaListItemDto
{
    public int Id { get; init; }
    public string Kod { get; init; }
    public string Baslik { get; init; }
    public decimal Deger { get; init; }
    public bool Aktif { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, KampanyalarOperationClaims.Read)]
[Logged]
public sealed record GetByIdKampanyaQuery(int Id) : IQuery<Result<GetByIdKampanyaResponse, DomainError>>;

internal sealed class GetByIdKampanyaQueryHandler : IQueryHandler<GetByIdKampanyaQuery, Result<GetByIdKampanyaResponse, DomainError>>
{
    private readonly IKampanyaRepository _kampanyaRepository;
    private readonly KampanyaBusinessRules _kampanyaBusinessRules;

    public GetByIdKampanyaQueryHandler(
        IKampanyaRepository kampanyaRepository,
        KampanyaBusinessRules kampanyaBusinessRules)
    {
        _kampanyaRepository = kampanyaRepository;
        _kampanyaBusinessRules = kampanyaBusinessRules;
    }

    public async ValueTask<Result<GetByIdKampanyaResponse, DomainError>> HandleAsync(
        GetByIdKampanyaQuery request,
        CancellationToken cancellationToken)
    {
        Kampanya? entity = await _kampanyaRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _kampanyaBusinessRules.KampanyaShouldExistWhenSelected(entity);

        return Result<GetByIdKampanyaResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, KampanyalarOperationClaims.Read)]
[Logged]
public sealed record GetListKampanyaQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListKampanyaListItemDto>, DomainError>>;

internal sealed class GetListKampanyaQueryHandler : IQueryHandler<GetListKampanyaQuery, Result<GetListResponse<GetListKampanyaListItemDto>, DomainError>>
{
    private readonly IKampanyaRepository _kampanyaRepository;

    public GetListKampanyaQueryHandler(IKampanyaRepository kampanyaRepository)
    {
        _kampanyaRepository = kampanyaRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListKampanyaListItemDto>, DomainError>> HandleAsync(
        GetListKampanyaQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _kampanyaRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListKampanyaListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
