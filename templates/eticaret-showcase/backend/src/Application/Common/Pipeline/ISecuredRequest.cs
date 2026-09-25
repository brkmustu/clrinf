namespace EticaretApp.Application.Common.Pipeline;

public interface ISecuredRequest
{
    public string[] Roles { get; }
}
