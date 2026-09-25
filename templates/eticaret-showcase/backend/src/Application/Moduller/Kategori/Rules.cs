using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Features.Kategoriler;

public static class KategoriBusinessMessages
{
    public const string SectionName = "Kategori";
    public const string KategoriNotExists = "Kategori bulunamadı.";
    public const string KategoriAlreadyExists = "Bu Kategori zaten mevcut.";
}

public class KategoriBusinessRules : BaseBusinessRules
{
    private readonly IKategoriRepository _kategoriRepository;

    public KategoriBusinessRules(IKategoriRepository kategoriRepository)
    {
        _kategoriRepository = kategoriRepository;
    }

    public async Task KategoriShouldExistWhenSelected(Kategori? kategori)
    {
        if (kategori == null)
            throw new BusinessException(KategoriBusinessMessages.KategoriNotExists);
    }

    public async Task KategoriIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Kategori? kategori = await _kategoriRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await KategoriShouldExistWhenSelected(kategori);
    }
}
