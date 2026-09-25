using System.Collections.Generic;
using EticaretApp.Application.Common.Paging;

namespace EticaretApp.Application.Common.Responses;

public class GetListResponse<T> : IResponse
{
    private IList<T>? _items;

    public IList<T> Items
    {
        get => _items ??= new List<T>();
        set => _items = value;
    }

    public int Index { get; set; }
    public int Size { get; set; }
    public int Count { get; set; }
    public int Pages { get; set; }
    public bool HasPrevious { get; set; }
    public bool HasNext { get; set; }
}
