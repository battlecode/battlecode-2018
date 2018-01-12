//------------------------------------------------------------------------------
// <auto-generated />
//
// This file was automatically generated by SWIG (http://www.swig.org).
// Version 3.0.8
//
// Do not make changes to this file unless you know what you are doing--modify
// the SWIG interface file instead.
//------------------------------------------------------------------------------


public class InitialTurnApplication : global::System.IDisposable {
  private global::System.Runtime.InteropServices.HandleRef swigCPtr;
  protected bool swigCMemOwn;

  internal InitialTurnApplication(global::System.IntPtr cPtr, bool cMemoryOwn) {
    swigCMemOwn = cMemoryOwn;
    swigCPtr = new global::System.Runtime.InteropServices.HandleRef(this, cPtr);
  }

  internal static global::System.Runtime.InteropServices.HandleRef getCPtr(InitialTurnApplication obj) {
    return (obj == null) ? new global::System.Runtime.InteropServices.HandleRef(null, global::System.IntPtr.Zero) : obj.swigCPtr;
  }

  ~InitialTurnApplication() {
    Dispose();
  }

  public virtual void Dispose() {
    lock(this) {
      if (swigCPtr.Handle != global::System.IntPtr.Zero) {
        if (swigCMemOwn) {
          swigCMemOwn = false;
          bcPINVOKE.delete_InitialTurnApplication(swigCPtr);
        }
        swigCPtr = new global::System.Runtime.InteropServices.HandleRef(null, global::System.IntPtr.Zero);
      }
      global::System.GC.SuppressFinalize(this);
    }
  }

  public InitialTurnApplication() : this(bcPINVOKE.new_InitialTurnApplication(), true) {
    if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
  }

  public StartTurnMessage start_turn {
    set {
      bcPINVOKE.InitialTurnApplication_start_turn_set(swigCPtr, StartTurnMessage.getCPtr(value));
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    } 
    get {
      global::System.IntPtr cPtr = bcPINVOKE.InitialTurnApplication_start_turn_get(swigCPtr);
      StartTurnMessage ret = (cPtr == global::System.IntPtr.Zero) ? null : new StartTurnMessage(cPtr, false);
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
      return ret;
    } 
  }

  public ViewerKeyframe viewer {
    set {
      bcPINVOKE.InitialTurnApplication_viewer_set(swigCPtr, ViewerKeyframe.getCPtr(value));
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
    } 
    get {
      global::System.IntPtr cPtr = bcPINVOKE.InitialTurnApplication_viewer_get(swigCPtr);
      ViewerKeyframe ret = (cPtr == global::System.IntPtr.Zero) ? null : new ViewerKeyframe(cPtr, false);
      if (bcPINVOKE.SWIGPendingException.Pending) throw bcPINVOKE.SWIGPendingException.Retrieve();
      return ret;
    } 
  }

}
