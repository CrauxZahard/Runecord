using System;
using System.Runtime.InteropServices;
using System.Threading;

public static class WasapiPlay
{
    const int Render = 0;
    const int RoleConsole = 0;
    const int Shared = 0;
    const int ClsCtxAll = 23;
    const uint AutoConvert = 0x80000000;
    const uint SrcDefaultQuality = 0x08000000;

    [ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
    class MMDeviceEnumeratorComObject { }

    [ComImport, Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IMMDeviceEnumerator
    {
        int EnumAudioEndpoints(int dataFlow, int stateMask, out IMMDeviceCollection devices);
        int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice endpoint);
        int GetDevice(string id, out IMMDevice device);
        int RegisterEndpointNotificationCallback(IntPtr client);
        int UnregisterEndpointNotificationCallback(IntPtr client);
    }

    [ComImport, Guid("0BD7A1BE-7A1A-44DB-8397-CC5392387B5E"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IMMDeviceCollection
    {
        int GetCount(out uint count);
        int Item(uint index, out IMMDevice device);
    }

    [ComImport, Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IMMDevice
    {
        int Activate(ref Guid iid, int clsCtx, IntPtr activationParams, [MarshalAs(UnmanagedType.IUnknown)] out object instance);
        int OpenPropertyStore(int access, out IPropertyStore store);
        int GetId([MarshalAs(UnmanagedType.LPWStr)] out string id);
        int GetState(out int state);
    }

    [ComImport, Guid("886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IPropertyStore
    {
        int GetCount(out uint count);
        int GetAt(uint index, out PROPERTYKEY key);
        int GetValue(ref PROPERTYKEY key, out PROPVARIANT value);
        int SetValue(ref PROPERTYKEY key, ref PROPVARIANT value);
        int Commit();
    }

    [StructLayout(LayoutKind.Sequential)]
    struct PROPERTYKEY
    {
        public Guid formatId;
        public int propertyId;
    }

    [StructLayout(LayoutKind.Explicit)]
    struct PROPVARIANT
    {
        [FieldOffset(0)] public ushort vt;
        [FieldOffset(8)] public IntPtr pointer;
    }

    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    struct WAVEFORMATEX
    {
        public ushort formatTag;
        public ushort channels;
        public uint samplesPerSec;
        public uint avgBytesPerSec;
        public ushort blockAlign;
        public ushort bitsPerSample;
        public ushort cbSize;
    }

    [ComImport, Guid("1CB9AD4C-DBFA-4C32-B178-C2F568A703B2"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IAudioClient
    {
        int Initialize(int shareMode, uint streamFlags, long bufferDuration, long periodicity, IntPtr format, IntPtr sessionGuid);
        int GetBufferSize(out uint frames);
        int GetStreamLatency(out long latency);
        int GetCurrentPadding(out uint frames);
        int IsFormatSupported(int shareMode, IntPtr format, out IntPtr closest);
        int GetMixFormat(out IntPtr format);
        int GetDevicePeriod(out long defaultPeriod, out long minimumPeriod);
        int Start();
        int Stop();
        int Reset();
        int SetEventHandle(IntPtr handle);
        int GetService(ref Guid iid, [MarshalAs(UnmanagedType.IUnknown)] out object service);
    }

    [ComImport, Guid("F294ACFC-3146-4483-A7BF-ADDCA7C260E2"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface IAudioRenderClient
    {
        int GetBuffer(uint frames, out IntPtr data);
        int ReleaseBuffer(uint frames, uint flags);
    }

    static string FriendlyName(IMMDevice device)
    {
        try
        {
            if (device.OpenPropertyStore(0, out IPropertyStore store) != 0) return "";
            PROPERTYKEY key = new PROPERTYKEY
            {
                formatId = new Guid("a45c254e-df1c-4efd-8020-67d146a850e0"),
                propertyId = 14
            };
            if (store.GetValue(ref key, out PROPVARIANT value) != 0) return "";
            if (value.vt == 31 && value.pointer != IntPtr.Zero)
                return Marshal.PtrToStringUni(value.pointer) ?? "";
            return "";
        }
        catch { return ""; }
    }

    static IMMDevice FindDevice(IMMDeviceEnumerator enumerator, string wanted)
    {
        if (string.IsNullOrWhiteSpace(wanted))
        {
            enumerator.GetDefaultAudioEndpoint(Render, RoleConsole, out IMMDevice endpoint);
            return endpoint;
        }
        if (enumerator.EnumAudioEndpoints(Render, 1, out IMMDeviceCollection collection) != 0)
            return null;
        collection.GetCount(out uint count);
        for (uint index = 0; index < count; index++)
        {
            if (collection.Item(index, out IMMDevice device) != 0) continue;
            string name = FriendlyName(device);
            if (name.Equals(wanted, StringComparison.OrdinalIgnoreCase) ||
                name.IndexOf(wanted, StringComparison.OrdinalIgnoreCase) >= 0)
                return device;
            if (wanted.Equals("Default", StringComparison.OrdinalIgnoreCase))
            {
                enumerator.GetDefaultAudioEndpoint(Render, RoleConsole, out IMMDevice def);
                return def;
            }
        }
        return null;
    }

    public static int Main(string[] args)
    {
        string wanted = args.Length > 0 ? args[0] : "";
        float volume = 1.0f;
        if (args.Length > 1) float.TryParse(args[1], System.Globalization.NumberStyles.Float,
            System.Globalization.CultureInfo.InvariantCulture, out volume);

        IMMDeviceEnumerator enumerator = (IMMDeviceEnumerator)new MMDeviceEnumeratorComObject();
        IMMDevice device = FindDevice(enumerator, wanted);
        if (device == null)
        {
            Console.Error.WriteLine("device not found: " + wanted);
            return 3;
        }
        string name = FriendlyName(device);
        Guid clientId = typeof(IAudioClient).GUID;
        if (device.Activate(ref clientId, ClsCtxAll, IntPtr.Zero, out object clientObject) != 0)
        {
            Console.Error.WriteLine("activate failed");
            return 2;
        }
        IAudioClient client = (IAudioClient)clientObject;
        WAVEFORMATEX format = new WAVEFORMATEX
        {
            formatTag = 1,
            channels = 1,
            samplesPerSec = 48000,
            avgBytesPerSec = 96000,
            blockAlign = 2,
            bitsPerSample = 16,
            cbSize = 0
        };
        IntPtr formatPtr = Marshal.AllocHGlobal(Marshal.SizeOf(format));
        Marshal.StructureToPtr(format, formatPtr, false);
        int hr = client.Initialize(Shared, AutoConvert | SrcDefaultQuality, 10_000_000, 0, formatPtr, IntPtr.Zero);
        if (hr != 0)
        {
            Console.Error.WriteLine("initialize failed hr=" + hr);
            return 2;
        }
        Guid renderId = typeof(IAudioRenderClient).GUID;
        if (client.GetService(ref renderId, out object renderObject) != 0)
        {
            Console.Error.WriteLine("render service failed");
            return 2;
        }
        IAudioRenderClient render = (IAudioRenderClient)renderObject;
        client.GetBufferSize(out uint bufferFrames);
        client.Start();
        Console.Error.WriteLine($"device={name} frames={bufferFrames} volume={volume}");

        var input = Console.OpenStandardInput();
        byte[] readBuffer = new byte[1920];
        byte[] output = new byte[1920];
        bool running = true;
        while (running)
        {
            client.GetCurrentPadding(out uint padding);
            uint available = bufferFrames - padding;
            if (available == 0)
            {
                Thread.Sleep(8);
                continue;
            }
            int wantedBytes = (int)Math.Min(available * 2, (uint)readBuffer.Length);
            int read = input.Read(readBuffer, 0, wantedBytes);
            if (read <= 0) break;
            int frames = (read + 1) / 2;
            int bytes = frames * 2;
            Array.Clear(output, 0, bytes);
            for (int i = 0; i < read; i += 2)
            {
                short sample = (short)(readBuffer[i] | ((i + 1 < read ? readBuffer[i + 1] : 0) << 8));
                int scaled = (int)Math.Round(sample * volume);
                scaled = Math.Max(-32768, Math.Min(32767, scaled));
                output[i] = (byte)(scaled & 0xFF);
                output[i + 1] = (byte)((scaled >> 8) & 0xFF);
            }
            if (render.GetBuffer((uint)frames, out IntPtr destination) == 0)
            {
                Marshal.Copy(output, 0, destination, bytes);
                render.ReleaseBuffer((uint)frames, 0);
            }
        }
        client.Stop();
        Console.Error.WriteLine("playback stopped");
        return 0;
    }
}
