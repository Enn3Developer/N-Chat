namespace Client.ViewModels;

public class MainViewModel : ViewModelBase
{
    public MainViewModel()
    {
    }

    public void OnLoaded()
    {
        MainWindowViewModel.Instance.InitSpacetimeDb();
    }
}