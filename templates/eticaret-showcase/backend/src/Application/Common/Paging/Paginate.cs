using System;
using System.Collections.Generic;
using System.Linq;

namespace EticaretApp.Application.Common.Paging;

public class Paginate<T> : IPaginate<T>
{
    public int From { get; set; }
    public int Index { get; set; }
    public int Size { get; set; }
    public int Count { get; set; }
    public int Pages { get; set; }
    public IList<T> Items { get; set; } = Array.Empty<T>();
    public bool HasPrevious => Index - From > 0;
    public bool HasNext => Index - From + 1 < Pages;

    public Paginate() { }

    public Paginate(IEnumerable<T> source, int index, int size, int from = 0)
    {
        var enumerable = source as T[] ?? source.ToArray();
        
        From = from;
        Index = index;
        Size = size;
        Count = enumerable.Length;
        Pages = (int)Math.Ceiling(Count / (double)Size);
        Items = enumerable.Skip((Index - From) * Size).Take(Size).ToList();
    }
}
