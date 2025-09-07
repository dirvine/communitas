const WORDS = [
  'able','about','above','access','action','adapt','add','agent','agree','ahead','alpha','angel','answer','apple','area','army','aspect','asset','audio','auto',
  'awake','axis','baby','back','basic','beach','begin','believe','beta','beyond','bike','bird','black','block','blue','board','body','brain','brave','bridge',
  'bright','bring','build','cabin','cable','camp','canal','candy','carry','chain','chair','chalk','change','charge','chase','check','cherry','chief','child','claim',
  'class','clear','clock','cloud','coach','coast','code','color','craft','crane','crash','crazy','cream','create','credit','crowd','curve','cycle','dance','dark',
  'data','deal','deer','delta','desert','device','dinner','direct','dirt','dive','doctor','dollar','domain','draft','dragon','dream','dress','drive','earth','edge',
  'effect','eight','elastic','elder','electric','element','elite','energy','engine','enjoy','enter','equal','error','escape','estate','ethics','even','event','every',
  'exact','example','excess','exchange','excite','expect','extend','fabric','factor','fair','faith','family','famous','fancy','farm','fashion','fast','father','feature',
  'field','final','finger','finish','fire','first','fix','flame','flight','flower','focus','follow','force','forest','forget','forward','frame','fresh','friend',
  'frozen','future','galaxy','garden','gauge','general','gentle','giant','glass','globe','gold','grace','grain','grand','grant','graph','grape','grass','green',
  'grid','group','guard','guess','guide','habit','hammer','happy','harbor','hard','harvest','hazard','heart','heavy','height','hello','hero','hidden','high',
  'honey','honor','horse','hotel','house','human','humble','ideal','image','impact','index','inform','inject','inner','input','inside','inspire','intend','island',
  'ivory','jacket','jelly','jewel','jungle','junior','keen','kernel','kettle','key','king','knock','label','ladder','lady','land','laser','later','latin','layer',
  'leader','learn','legend','lemon','level','light','limit','linear','lion','liquid','logic','lunar','machine','magic','major','mango','maple','march','marine',
  'market','mask','master','matter','meadow','media','member','memory','metal','meter','middle','mineral','minute','mirror','mobile','model','modern','money','monster',
  'moral','motor','mountain','music','narrow','nation','native','nature','navy','near','neat','neck','nerve','neutral','never','night','noble','north','novel',
  'object','ocean','offer','office','olive','onion','open','opera','orange','orbit','order','organ','other','owner','oxygen','package','paddle','page','palm',
  'paper','parent','park','party','pass','path','peace','pearl','people','pepper','perfect','phone','photo','phrase','piano','piece','pine','pink','planet',
  'plastic','please','plenty','poetry','point','polar','police','popular','portion','position','potato','praise','press','pretty','price','pride','primary','print','prize',
  'profit','program','project','promise','proper','prosper','protect','public','pupil','purple','puzzle','queen','quiet','radio','raise','rally','rapid','rare','raven',
  'ready','reason','record','red','reduce','refine','region','relax','remain','remark','remote','reserve','resist','resource','result','reveal','review','reward','ridge',
  'river','robot','rocket','romance','rose','rough','round','route','royal','safety','salmon','sample','satisfy','satoshi','scale','scene','school','science',
  'screen','script','search','season','second','secret','seed','select','sensor','shadow','share','shift','shine','short','shoulder','silent','silver','simple','single',
  'sister','sketch','skill','sleep','smart','smile','snow','society','solar','solid','solution','sound','space','spark','speak','special','spice','spider','spirit',
  'split','sport','spread','spring','square','stable','stamp','stand','star','start','state','station','steam','steel','step','stone','storm','story','street',
  'strong','studio','sugar','summer','sun','sunset','supply','sure','survey','switch','symbol','system','table','tactic','tail','talent','target','taste','teach',
  'team','tech','temple','thank','theory','thing','thought','thunder','tiger','timber','tiny','token','tomato','topic','torch','tower','track','trade','train',
  'travel','treat','tree','trend','tribe','tropic','trouble','trust','truth','tunnel','uncover','unfair','unify','unique','unit','unlock','update','upper','urban',
  'useful','vacuum','valley','value','velvet','venture','verse','very','video','village','violet','vision','vital','voice','volume','voyage','wagon','water','wealth',
  'weapon','weather','web','welcome','west','whale','wheel','white','wild','will','window','winter','wire','wisdom','wolf','woman','wonder','world','yellow',
  'young','zebra'
]

export function generateMnemonic(count = 12): string[] {
  const out: string[] = []
  const rnd = (n: number) => crypto.getRandomValues(new Uint32Array(1))[0] % n
  for (let i = 0; i < count; i++) {
    out.push(WORDS[rnd(WORDS.length)])
  }
  return out
}

export function mnemonicToSeed(mnemonic: string[]): Uint8Array {
  const data = mnemonic.join(' ')
  return new TextEncoder().encode(data)
}

