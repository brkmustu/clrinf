using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Moduller.Sepetler;

public static class SepetMappingExtensions
{
    public static Sepet ToEntity(this CreateSepetCommand command) =>
        new()
        {
            KullaniciId = command.KullaniciId,
            ToplamTutar = command.ToplamTutar,
            IndirimTutari = command.IndirimTutari,
            Kilitli = command.Kilitli,
        };

    public static CreatedSepetResponse ToCreatedResponse(this Sepet sepet) =>
        new()
        {
            Id = sepet.Id,
            KullaniciId = sepet.KullaniciId,
            ToplamTutar = sepet.ToplamTutar,
            IndirimTutari = sepet.IndirimTutari,
            Kilitli = sepet.Kilitli,
        };

    public static Sepet ToEntity(this UpdateSepetCommand command) =>
        new()
        {
            Id = command.Id,
            KullaniciId = command.KullaniciId,
            ToplamTutar = command.ToplamTutar,
            IndirimTutari = command.IndirimTutari,
            Kilitli = command.Kilitli,
        };

    public static UpdatedSepetResponse ToUpdatedResponse(this Sepet sepet) =>
        new()
        {
            Id = sepet.Id,
            KullaniciId = sepet.KullaniciId,
            ToplamTutar = sepet.ToplamTutar,
            IndirimTutari = sepet.IndirimTutari,
            Kilitli = sepet.Kilitli,
        };

    public static DeletedSepetResponse ToDeletedResponse(this Sepet sepet) =>
        new() { Id = sepet.Id };

    public static GetByIdSepetResponse ToGetByIdResponse(this Sepet sepet) =>
        new()
        {
            Id = sepet.Id,
            KullaniciId = sepet.KullaniciId,
            ToplamTutar = sepet.ToplamTutar,
            IndirimTutari = sepet.IndirimTutari,
            Kilitli = sepet.Kilitli,
        };

    public static GetListSepetListItemDto ToListItemDto(this Sepet sepet) =>
        new()
        {
            Id = sepet.Id,
            KullaniciId = sepet.KullaniciId,
            ToplamTutar = sepet.ToplamTutar,
            IndirimTutari = sepet.IndirimTutari,
            Kilitli = sepet.Kilitli,
        };

    public static GetListResponse<GetListSepetListItemDto> ToPaginateResponse(this IList<Sepet> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
