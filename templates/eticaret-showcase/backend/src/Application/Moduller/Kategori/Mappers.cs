using System.Collections.Generic;
using System.Linq;
using EticaretApp.Domain.Entities;
using EticaretApp.Application.Common.Responses;

namespace EticaretApp.Application.Features.Kategoriler;

public static class KategoriMappingExtensions
{
    public static Kategori ToEntity(this CreateKategoriCommand command) =>
        new()
        {
        };

    public static CreatedKategoriResponse ToCreatedResponse(this Kategori kategori) =>
        new()
        {
            Id = kategori.Id,
        };

    public static Kategori ToEntity(this UpdateKategoriCommand command) =>
        new()
        {
            Id = command.Id,
        };

    public static UpdatedKategoriResponse ToUpdatedResponse(this Kategori kategori) =>
        new()
        {
            Id = kategori.Id,
        };

    public static DeletedKategoriResponse ToDeletedResponse(this Kategori kategori) =>
        new() { Id = kategori.Id };

    public static GetByIdKategoriResponse ToGetByIdResponse(this Kategori kategori) =>
        new()
        {
            Id = kategori.Id,
            Ad = kategori.Ad,
            Slug = kategori.Slug,
            Sira = kategori.Sira,
            Aktif = kategori.Aktif
        };

    public static GetListKategoriListItemDto ToListItemDto(this Kategori kategori) =>
        new()
        {
            Id = kategori.Id,
            Ad = kategori.Ad,
            Slug = kategori.Slug,
            Sira = kategori.Sira,
            Aktif = kategori.Aktif
        };

    public static GetListResponse<GetListKategoriListItemDto> ToPaginateResponse(this IList<Kategori> items) =>
        new()
        {
            Items = items.Select(x => x.ToListItemDto()).ToList(),
            Count = items.Count,
            Index = 0,
            Size = items.Count,
            Pages = 1
        };
}
