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
using static EticaretApp.Application.Moduller.Sepetler.SepetlerOperationClaims;

namespace EticaretApp.Application.Moduller.Sepetler;

#region Query Responses & DTOs

public sealed record GetByIdSepetResponse
{
    public int Id { get; init; }
    public string KullaniciId { get; init; }
    public decimal ToplamTutar { get; init; }
    public decimal IndirimTutari { get; init; }
    public bool Kilitli { get; init; }
}

public sealed record GetListSepetListItemDto
{
    public int Id { get; init; }
    public string KullaniciId { get; init; }
    public decimal ToplamTutar { get; init; }
    public decimal IndirimTutari { get; init; }
    public bool Kilitli { get; init; }
}

#endregion

#region GetById Query & Handler

[RequireAuthorization(Admin, Read, SepetlerOperationClaims.Read)]
[Logged]
public sealed record GetByIdSepetQuery(int Id) : IQuery<Result<GetByIdSepetResponse, DomainError>>;

internal sealed class GetByIdSepetQueryHandler : IQueryHandler<GetByIdSepetQuery, Result<GetByIdSepetResponse, DomainError>>
{
    private readonly ISepetRepository _sepetRepository;
    private readonly SepetBusinessRules _sepetBusinessRules;

    public GetByIdSepetQueryHandler(
        ISepetRepository sepetRepository,
        SepetBusinessRules sepetBusinessRules)
    {
        _sepetRepository = sepetRepository;
        _sepetBusinessRules = sepetBusinessRules;
    }

    public async ValueTask<Result<GetByIdSepetResponse, DomainError>> HandleAsync(
        GetByIdSepetQuery request,
        CancellationToken cancellationToken)
    {
        Sepet? entity = await _sepetRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await _sepetBusinessRules.SepetShouldExistWhenSelected(entity);

        return Result<GetByIdSepetResponse, DomainError>.Success(entity!.ToGetByIdResponse());
    }
}

#endregion

#region GetList Query & Handler

[RequireAuthorization(Admin, Read, SepetlerOperationClaims.Read)]
[Logged]
public sealed record GetListSepetQuery(PageRequest PageRequest) : IQuery<Result<GetListResponse<GetListSepetListItemDto>, DomainError>>;

internal sealed class GetListSepetQueryHandler : IQueryHandler<GetListSepetQuery, Result<GetListResponse<GetListSepetListItemDto>, DomainError>>
{
    private readonly ISepetRepository _sepetRepository;

    public GetListSepetQueryHandler(ISepetRepository sepetRepository)
    {
        _sepetRepository = sepetRepository;
    }

    public async ValueTask<Result<GetListResponse<GetListSepetListItemDto>, DomainError>> HandleAsync(
        GetListSepetQuery request,
        CancellationToken cancellationToken)
    {
        var entities = await _sepetRepository.GetListAsync(
            index: request.PageRequest.PageIndex,
            size: request.PageRequest.PageSize,
            enableTracking: false,
            cancellationToken: cancellationToken
        );

        return Result<GetListResponse<GetListSepetListItemDto>, DomainError>.Success(entities.Items.ToPaginateResponse());
    }
}

#endregion
