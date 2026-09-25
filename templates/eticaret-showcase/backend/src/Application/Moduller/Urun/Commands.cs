using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Moduller.Urunlar.UrunlarOperationClaims;

namespace EticaretApp.Application.Moduller.Urunlar;

#region Responses

public sealed record CreatedUrunResponse
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

public sealed record UpdatedUrunResponse
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

public sealed record DeletedUrunResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, UrunlarOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateUrunCommand(
    string Sku, 
    string Baslik, 
    string? Aciklama, 
    decimal BirimFiyat, 
    string ParaBirimi, 
    string KategoriYolu, 
    int MusaitStok, 
    bool Yayinda
) : ICommand<Result<CreatedUrunResponse, DomainError>>;

public class CreateUrunCommandValidator : AbstractValidator<CreateUrunCommand>
{
    public CreateUrunCommandValidator()
    {
    }
}

internal sealed class CreateUrunCommandHandler : ICommandHandler<CreateUrunCommand, Result<CreatedUrunResponse, DomainError>>
{
    private readonly IUrunRepository _urunRepository;
    private readonly UrunBusinessRules _urunBusinessRules;

    public CreateUrunCommandHandler(
        IUrunRepository urunRepository,
        UrunBusinessRules urunBusinessRules)
    {
        _urunRepository = urunRepository;
        _urunBusinessRules = urunBusinessRules;
    }

    public async ValueTask<Result<CreatedUrunResponse, DomainError>> HandleAsync(
        CreateUrunCommand request,
        CancellationToken cancellationToken)
    {
        Urun entity = request.ToEntity();
        await _urunRepository.AddAsync(entity);
        return Result<CreatedUrunResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, UrunlarOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateUrunCommand(
    int Id,
    string Sku, 
    string Baslik, 
    string? Aciklama, 
    decimal BirimFiyat, 
    string ParaBirimi, 
    string KategoriYolu, 
    int MusaitStok, 
    bool Yayinda
) : ICommand<Result<UpdatedUrunResponse, DomainError>>;

public class UpdateUrunCommandValidator : AbstractValidator<UpdateUrunCommand>
{
    public UpdateUrunCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateUrunCommandHandler : ICommandHandler<UpdateUrunCommand, Result<UpdatedUrunResponse, DomainError>>
{
    private readonly IUrunRepository _urunRepository;
    private readonly UrunBusinessRules _urunBusinessRules;

    public UpdateUrunCommandHandler(
        IUrunRepository urunRepository,
        UrunBusinessRules urunBusinessRules)
    {
        _urunRepository = urunRepository;
        _urunBusinessRules = urunBusinessRules;
    }

    public async ValueTask<Result<UpdatedUrunResponse, DomainError>> HandleAsync(
        UpdateUrunCommand request,
        CancellationToken cancellationToken)
    {
        Urun? entity = await _urunRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _urunBusinessRules.UrunShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _urunRepository.UpdateAsync(entity);
        return Result<UpdatedUrunResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, UrunlarOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteUrunCommand(int Id) : ICommand<Result<DeletedUrunResponse, DomainError>>;

public class DeleteUrunCommandValidator : AbstractValidator<DeleteUrunCommand>
{
    public DeleteUrunCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteUrunCommandHandler : ICommandHandler<DeleteUrunCommand, Result<DeletedUrunResponse, DomainError>>
{
    private readonly IUrunRepository _urunRepository;
    private readonly UrunBusinessRules _urunBusinessRules;

    public DeleteUrunCommandHandler(
        IUrunRepository urunRepository,
        UrunBusinessRules urunBusinessRules)
    {
        _urunRepository = urunRepository;
        _urunBusinessRules = urunBusinessRules;
    }

    public async ValueTask<Result<DeletedUrunResponse, DomainError>> HandleAsync(
        DeleteUrunCommand request,
        CancellationToken cancellationToken)
    {
        Urun? entity = await _urunRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _urunBusinessRules.UrunShouldExistWhenSelected(entity);

        await _urunRepository.DeleteAsync(entity!);
        return Result<DeletedUrunResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
