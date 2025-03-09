using ReactiveUI;

namespace Client.ViewModels;

public class MainWindowViewModel : ViewModelBase
{
    public ViewModelBase ContentViewModel
    {
        get;
        private set => this.RaiseAndSetIfChanged(ref field, value);
    } = null!;

    public MainWindowViewModel()
    {
        // TODO: set ContentViewModel
    }
}