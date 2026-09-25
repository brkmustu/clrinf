using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using EticaretApp.Application.Common.Attributes;
using EticaretApp.Application.Common.Functional;
using EticaretApp.Application.Common.Pipeline;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Common.Exceptions;
using EticaretApp.Domain.Entities;
using static EticaretApp.Application.Features.Kategoriler.KategorilerOperationClaims;

namespace EticaretApp.Application.Features.Kategoriler;

#region Responses

public sealed record CreatedKategoriResponse
{
    public int Id { get; init; }
}

public sealed record UpdatedKategoriResponse
{
    public int Id { get; init; }
}

public sealed record DeletedKategoriResponse
{
    public int Id { get; init; }
}

#endregion

#region Create Command & Handler

[RequireAuthorization(Admin, Write, KategorilerOperationClaims.Create)]
[Logged]
[Transactional]
public sealed record CreateKategoriCommand(
) : ICommand<Result<CreatedKategoriResponse, DomainError>>;

public class CreateKategoriCommandValidator : AbstractValidator<CreateKategoriCommand>
{
    public CreateKategoriCommandValidator()
    {
    }
}

internal sealed class CreateKategoriCommandHandler : ICommandHandler<CreateKategoriCommand, Result<CreatedKategoriResponse, DomainError>>
{
    private readonly IKategoriRepository _kategoriRepository;
    private readonly KategoriBusinessRules _kategoriBusinessRules;

    public CreateKategoriCommandHandler(
        IKategoriRepository kategoriRepository,
        KategoriBusinessRules kategoriBusinessRules)
    {
        _kategoriRepository = kategoriRepository;
        _kategoriBusinessRules = kategoriBusinessRules;
    }

    public async ValueTask<Result<CreatedKategoriResponse, DomainError>> HandleAsync(
        CreateKategoriCommand request,
        CancellationToken cancellationToken)
    {
        Kategori entity = request.ToEntity();
        await _kategoriRepository.AddAsync(entity);
        return Result<CreatedKategoriResponse, DomainError>.Success(entity.ToCreatedResponse());
    }
}

#endregion

#region Update Command & Handler

[RequireAuthorization(Admin, Write, KategorilerOperationClaims.Update)]
[Logged]
[Transactional]
public sealed record UpdateKategoriCommand(
    int Id) : ICommand<Result<UpdatedKategoriResponse, DomainError>>;

public class UpdateKategoriCommandValidator : AbstractValidator<UpdateKategoriCommand>
{
    public UpdateKategoriCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class UpdateKategoriCommandHandler : ICommandHandler<UpdateKategoriCommand, Result<UpdatedKategoriResponse, DomainError>>
{
    private readonly IKategoriRepository _kategoriRepository;
    private readonly KategoriBusinessRules _kategoriBusinessRules;

    public UpdateKategoriCommandHandler(
        IKategoriRepository kategoriRepository,
        KategoriBusinessRules kategoriBusinessRules)
    {
        _kategoriRepository = kategoriRepository;
        _kategoriBusinessRules = kategoriBusinessRules;
    }

    public async ValueTask<Result<UpdatedKategoriResponse, DomainError>> HandleAsync(
        UpdateKategoriCommand request,
        CancellationToken cancellationToken)
    {
        Kategori? entity = await _kategoriRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _kategoriBusinessRules.KategoriShouldExistWhenSelected(entity);

        entity = request.ToEntity();
        await _kategoriRepository.UpdateAsync(entity);
        return Result<UpdatedKategoriResponse, DomainError>.Success(entity.ToUpdatedResponse());
    }
}

#endregion

#region Delete Command & Handler

[RequireAuthorization(Admin, Write, KategorilerOperationClaims.Delete)]
[Logged]
[Transactional]
public sealed record DeleteKategoriCommand(int Id) : ICommand<Result<DeletedKategoriResponse, DomainError>>;

public class DeleteKategoriCommandValidator : AbstractValidator<DeleteKategoriCommand>
{
    public DeleteKategoriCommandValidator()
    {
        RuleFor(c => c.Id).NotEmpty();
    }
}

internal sealed class DeleteKategoriCommandHandler : ICommandHandler<DeleteKategoriCommand, Result<DeletedKategoriResponse, DomainError>>
{
    private readonly IKategoriRepository _kategoriRepository;
    private readonly KategoriBusinessRules _kategoriBusinessRules;

    public DeleteKategoriCommandHandler(
        IKategoriRepository kategoriRepository,
        KategoriBusinessRules kategoriBusinessRules)
    {
        _kategoriRepository = kategoriRepository;
        _kategoriBusinessRules = kategoriBusinessRules;
    }

    public async ValueTask<Result<DeletedKategoriResponse, DomainError>> HandleAsync(
        DeleteKategoriCommand request,
        CancellationToken cancellationToken)
    {
        Kategori? entity = await _kategoriRepository.GetAsync(
            predicate: x => x.Id.Equals(request.Id),
            cancellationToken: cancellationToken
        );
        await _kategoriBusinessRules.KategoriShouldExistWhenSelected(entity);

        await _kategoriRepository.DeleteAsync(entity!);
        return Result<DeletedKategoriResponse, DomainError>.Success(entity!.ToDeletedResponse());
    }
}

#endregion
