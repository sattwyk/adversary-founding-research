package main

import("bufio";"encoding/json";"fmt";"os";"os/exec";"strings";"syscall";"time")

func must(e error){if e!=nil{panic(e)}}
func mount(src,target,typ string){os.MkdirAll(target,0755);flags:=uintptr(0);if typ=="squashfs"{flags=syscall.MS_RDONLY};if e:=syscall.Mount(src,target,typ,flags,"");e!=nil{panic(fmt.Sprintf("mount %s -> %s (%s), uid=%d: %v",src,target,typ,os.Getuid(),e))}}
func mod(args ...string){out,e:=exec.Command("/usr/sbin/modprobe",args...).CombinedOutput();if e!=nil{panic(fmt.Sprint(args,string(out),e))}}
func emit(v any){b,_:=json.Marshal(v);fmt.Println("ADV:"+string(b));for{time.Sleep(time.Hour)}}
func syncdir(){d,e:=os.Open("/data");must(e);must(d.Sync());d.Close()}
func main(){
 defer func(){if e:=recover();e!=nil{emit(map[string]any{"error":fmt.Sprint(e)})}}()
 mount("proc","/proc","proc");mount("sysfs","/sys","sysfs");mount("devtmpfs","/dev","devtmpfs")
 mod("virtio_blk");mod("squashfs")
 for i:=0;i<50;i++{if _,e:=os.Stat("/dev/vdb");e==nil{break};time.Sleep(20*time.Millisecond)}
 mount("/dev/vdb","/modloop","squashfs")
 must(os.MkdirAll("/modroot/lib",0755));must(os.Symlink("/modloop/modules","/modroot/lib/modules"))
 mod("-d","/modroot","ext4")
 mode,which,impl:="","","file"
 raw,e:=os.ReadFile("/proc/cmdline");must(e)
 for _,x:=range strings.Fields(string(raw)){if strings.HasPrefix(x,"adv.mode="){mode=strings.TrimPrefix(x,"adv.mode=")};if strings.HasPrefix(x,"adv.case="){which=strings.TrimPrefix(x,"adv.case=")};if strings.HasPrefix(x,"adv.impl="){impl=strings.TrimPrefix(x,"adv.impl=")}}
 mount("/dev/vda","/data","ext4")
 if impl=="bbolt"{
  path:="/data/db";action:=mode;if action=="recover"{action="read"};if action=="baseline"{action="init"}
  cmd:=exec.Command("/bbolt-worker",action,path,which)
  if mode=="write"{
   stdin,e:=cmd.StdinPipe();must(e);defer stdin.Close();stdout,e:=cmd.StdoutPipe();must(e);cmd.Stderr=os.Stderr;must(cmd.Start());scan:=bufio.NewScanner(stdout);if !scan.Scan(){panic("no bbolt ack")};emit(map[string]any{"mode":mode,"impl":impl,"case":which,"ack":scan.Text()})
  }else{out,e:=cmd.CombinedOutput();must(e);if mode=="baseline"{syncdir()};emit(map[string]any{"mode":mode,"impl":impl,"case":which,"value":strings.TrimSpace(string(out))})}
 }
 switch mode{
 case "baseline":
  f,e:=os.OpenFile("/data/value",os.O_CREATE|os.O_RDWR,0600);must(e);_,e=f.Write([]byte("0"));must(e);must(f.Sync());must(f.Close());syncdir();emit(map[string]any{"mode":mode,"value":"0"})
 case "write":
  if which=="rename-synced"{
   f,e:=os.OpenFile("/data/next",os.O_CREATE|os.O_RDWR,0600);must(e);_,e=f.Write([]byte("1"));must(e);must(f.Sync());must(f.Close());must(os.Rename("/data/next","/data/value"));syncdir()
  }else{
   f,e:=os.OpenFile("/data/value",os.O_RDWR,0600);must(e);_,e=f.WriteAt([]byte("1"),0);must(e);if which=="synced"{must(f.Sync())}
  }
  emit(map[string]any{"mode":mode,"case":which,"ack":true})
 case "recover": b,e:=os.ReadFile("/data/value");must(e);emit(map[string]any{"mode":mode,"case":which,"value":string(b)})
 default:panic("unknown mode")
 }
}
