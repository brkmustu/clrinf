using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Features.Degerlendirmeler;

public static class DegerlendirmeMappingExtensions
{
    public static Degerlendirme ToEntity(this CreateDegerlendirmeCommand command) =>
        new()
        {
        };

    public static CreatedDegerlendirmeResponse ToCreatedResponse(this Degerlendirme degerlendirme) =>
        new()
        {
            Id = degerlendirme.Id,
        };

    public static Degerlendirme ToEntity(this UpdateDegerlendirmeCommand command) =>
        new()
        {
            Id = command.Id,
        };

    public static UpdatedDegerlendirmeResponse ToUpdatedResponse(this Degerlendirme degerlendirme) =>
        new()
        {
            Id = degerlendirme.Id,
        };

    public static DeletedDegerlendirmeResponse ToDeletedResponse(this Degerlendirme degerlendirme) =>
        new() { Id = degerlendirme.Id };

    public static GetByIdDegerlendirmeResponse ToGetByIdResponse(this Degerlendirme degerlendirme) =>
        new()
        {
            Id = degerlendirme.Id,
            UrunId = degerlendirme.UrunId,
            KullaniciId = degerlendirme.KullaniciId,
            Puan = degerlendirme.Puan,
            Yorum = degerlendirme.Yorum,
            Onaylandi = degerlendirme.Onaylandi
        };

    public static GetListDegerlendirmeListItemDto ToListItemDto(this Degerlendirme degerlendirme) =>
        new()
        {
            Id = degerlendirme.Id,
            UrunId = degerlendirme.UrunId,
            KullaniciId = degerlendirme.KullaniciId,
            Puan = degerlendirme.Puan,
            Yorum = degerlendirme.Yorum,
            Onaylandi = degerlendirme.Onaylandi
        };

    public static GetListResponse<GetListDegerlendirmeListItemDto> ToPaginateResponse(this IList<Degerlendirme> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
