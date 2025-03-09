using ReactiveUI;

namespace Client.ViewModels;

public class ViewModelBase : ReactiveObject
{
    public virtual void OnClose()
    {
    }
}