using Avalonia.Controls;
using Client.ViewModels;

namespace Client.Views;

public partial class MainWindow : Window
{
    public static MainWindow? Instance { get; private set; }

    public MainWindow()
    {
        InitializeComponent();
        Instance = this;
        Closing += (sender, args) =>
        {
            if (DataContext is MainWindowViewModel mainWindowViewModel)
                mainWindowViewModel.OnClose();
        };
    }
}