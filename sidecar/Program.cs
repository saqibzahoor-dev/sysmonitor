using System;
using System.Globalization;
using System.IO;
using System.Threading;
using LibreHardwareMonitor.Hardware;

namespace SysmonSensor
{
    internal sealed class UpdateVisitor : IVisitor
    {
        public void VisitComputer(IComputer computer) { computer.Traverse(this); }
        public void VisitHardware(IHardware hardware)
        {
            hardware.Update();
            foreach (var sub in hardware.SubHardware) sub.Accept(this);
        }
        public void VisitSensor(ISensor sensor) { }
        public void VisitParameter(IParameter parameter) { }
    }

    internal static class Program
    {
        private static int Main()
        {
            var computer = new Computer
            {
                IsCpuEnabled = true,
                IsGpuEnabled = true,
                IsMemoryEnabled = false,
                IsMotherboardEnabled = false,
                IsStorageEnabled = false,
                IsNetworkEnabled = false,
            };
            computer.Open();
            var visitor = new UpdateVisitor();

            Console.OutputEncoding = System.Text.Encoding.UTF8;
            using (var sw = new StreamWriter(Console.OpenStandardOutput()) { AutoFlush = true })
            {
                while (true)
                {
                    try
                    {
                        computer.Accept(visitor);
                        var ts = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();
                        float? cpuTemp = null;
                        float? cpuPower = null;
                        var gpuParts = new System.Text.StringBuilder();
                        bool firstGpu = true;

                        foreach (var hw in computer.Hardware)
                        {
                            if (hw.HardwareType == HardwareType.Cpu)
                            {
                                foreach (var s in hw.Sensors)
                                {
                                    if (s.SensorType == SensorType.Temperature &&
                                        (s.Name.Contains("Package") || s.Name.Contains("CCD1") || s.Name.Contains("Tctl")))
                                    {
                                        if (cpuTemp == null) cpuTemp = s.Value;
                                    }
                                    if (s.SensorType == SensorType.Power && s.Name.Contains("Package"))
                                    {
                                        if (cpuPower == null) cpuPower = s.Value;
                                    }
                                }
                            }
                            else if (hw.HardwareType == HardwareType.GpuAmd ||
                                     hw.HardwareType == HardwareType.GpuNvidia ||
                                     hw.HardwareType == HardwareType.GpuIntel)
                            {
                                float? load = null;
                                float? temp = null;
                                uint vramUsed = 0;
                                uint vramTotal = 0;
                                foreach (var s in hw.Sensors)
                                {
                                    if (s.SensorType == SensorType.Load && s.Name == "GPU Core") load = s.Value;
                                    if (s.SensorType == SensorType.Temperature && s.Name.StartsWith("GPU Core")) temp = s.Value;
                                    if (s.SensorType == SensorType.SmallData && s.Name == "GPU Memory Used") vramUsed = (uint)(s.Value ?? 0);
                                    if (s.SensorType == SensorType.SmallData && s.Name == "GPU Memory Total") vramTotal = (uint)(s.Value ?? 0);
                                }
                                if (!firstGpu) gpuParts.Append(',');
                                firstGpu = false;
                                gpuParts.Append("{\"name\":\"")
                                    .Append(Escape(hw.Name))
                                    .Append("\",\"load_pct\":").Append(Format(load))
                                    .Append(",\"temp_c\":").Append(Format(temp))
                                    .Append(",\"vram_used_mb\":").Append(vramUsed)
                                    .Append(",\"vram_total_mb\":").Append(vramTotal)
                                    .Append('}');
                            }
                        }

                        sw.Write("{\"ts\":");
                        sw.Write(ts);
                        sw.Write(",\"cpu\":{\"temp_c\":");
                        sw.Write(Format(cpuTemp));
                        sw.Write(",\"package_w\":");
                        sw.Write(Format(cpuPower));
                        sw.Write("},\"gpus\":[");
                        sw.Write(gpuParts);
                        sw.Write("]}\n");
                    }
                    catch (Exception ex)
                    {
                        Console.Error.WriteLine("sidecar error: " + ex.Message);
                    }
                    Thread.Sleep(1000);
                }
            }
        }

        private static string Format(float? v)
        {
            return v.HasValue ? v.Value.ToString("0.0", CultureInfo.InvariantCulture) : "null";
        }

        private static string Escape(string s)
        {
            return s.Replace("\\", "\\\\").Replace("\"", "\\\"");
        }
    }
}
