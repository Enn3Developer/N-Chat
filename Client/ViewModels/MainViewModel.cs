using System.Collections.ObjectModel;
using ReactiveUI;
using SpacetimeDB.Types;

namespace Client.ViewModels;

public class MainViewModel : ViewModelBase
{
    public ObservableCollection<ChannelViewModel> Channels => MainWindowViewModel.Instance.Channels;

    public MainViewModel()
    {
    }

    public void OnLoaded()
    {
        MainWindowViewModel.Instance.InitSpacetimeDb();
    }
}