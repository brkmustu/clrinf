using System.Threading;
using System.Threading.Tasks;
using EticaretApp.Application.Common.Rules;
using EticaretApp.Application.Common.Exceptions;
using EticaretApp.Application.Services.Repositories;
using EticaretApp.Domain.Entities;

namespace EticaretApp.Application.Features.Degerlendirmeler;

public static class DegerlendirmeBusinessMessages
{
    public const string SectionName = "Degerlendirme";
    public const string DegerlendirmeNotExists = "Degerlendirme bulunamadı.";
    public const string DegerlendirmeAlreadyExists = "Bu Degerlendirme zaten mevcut.";
}

public class DegerlendirmeBusinessRules : BaseBusinessRules
{
    private readonly IDegerlendirmeRepository _degerlendirmeRepository;

    public DegerlendirmeBusinessRules(IDegerlendirmeRepository degerlendirmeRepository)
    {
        _degerlendirmeRepository = degerlendirmeRepository;
    }

    public async Task DegerlendirmeShouldExistWhenSelected(Degerlendirme? degerlendirme)
    {
        if (degerlendirme == null)
            throw new BusinessException(DegerlendirmeBusinessMessages.DegerlendirmeNotExists);
    }

    public async Task DegerlendirmeIdShouldExistWhenSelected(int id, CancellationToken cancellationToken)
    {
        Degerlendirme? degerlendirme = await _degerlendirmeRepository.GetAsync(
            predicate: x => x.Id.Equals(id),
            enableTracking: false,
            cancellationToken: cancellationToken
        );
        await DegerlendirmeShouldExistWhenSelected(degerlendirme);
    }
}
